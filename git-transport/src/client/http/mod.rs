use crate::{client, client::git, Protocol, Service};
use git_features::pipe;
use std::{
    borrow::Cow,
    convert::Infallible,
    io::{BufRead, Read},
};

#[cfg(feature = "http-client-curl")]
pub(crate) mod curl;

mod traits;
pub use traits::{Error, GetResponse, Http, PostResponse};

#[cfg(feature = "http-client-curl")]
type HttpImpl = curl::Curl;

pub struct Transport {
    url: String,
    user_agent_header: &'static str,
    version: crate::Protocol,
    http: HttpImpl,
    service: Option<Service>,
    line_reader: git_packetline::Reader<pipe::Reader>,
    line_writer: git_packetline::Writer<pipe::Writer>,
}

impl Transport {
    pub fn new(url: &str, version: crate::Protocol) -> Self {
        let dummy = pipe::unidirectional(0);
        Transport {
            url: url.to_owned(),
            user_agent_header: concat!("User-Agent: git/oxide-", env!("CARGO_PKG_VERSION")),
            version,
            service: None,
            http: HttpImpl::new(),
            line_reader: git_packetline::Reader::new(dummy.1, None),
            line_writer: git_packetline::Writer::new(dummy.0),
        }
    }
}

impl client::Transport for Transport {}

fn append_url(base: &str, suffix: &str) -> String {
    if base.ends_with('/') {
        format!("{}{}", base, suffix)
    } else {
        format!("{}/{}", base, suffix)
    }
}

impl client::TransportSketch for Transport {
    fn handshake(&mut self, service: Service) -> Result<client::SetServiceResponse, client::Error> {
        let url = append_url(&self.url, &format!("info/refs?service={}", service.as_str()));
        let static_headers = [Cow::Borrowed(self.user_agent_header)];
        let mut dynamic_headers = Vec::<Cow<str>>::new();
        if self.version != Protocol::V1 {
            dynamic_headers.push(Cow::Owned(format!("Git-Protocol: version={}", self.version as usize)));
        }
        let GetResponse { headers, body } = self.http.get(&url, static_headers.iter().chain(&dynamic_headers))?;
        let wanted_content_type = format!("Content-Type: application/x-{}-advertisement", service.as_str());
        if !headers
            .lines()
            .collect::<Result<Vec<_>, _>>()?
            .iter()
            .any(|l| l == &wanted_content_type)
        {
            return Err(client::Error::Http(Error::Detail(format!(
                "Didn't find '{}' header to indicate 'smart' protocol, and 'dumb' protocol is not supported.",
                wanted_content_type
            ))));
        }

        self.line_reader.replace(body);

        let mut announced_service = String::new();
        self.line_reader.as_read().read_to_string(&mut announced_service)?;
        let expected_service_announcement = format!("# service={}", service.as_str());
        if announced_service.trim() != expected_service_announcement {
            return Err(client::Error::Http(Error::Detail(format!(
                "Expected to see {:?}, but got {:?}",
                expected_service_announcement,
                announced_service.trim()
            ))));
        }

        let (capabilities, refs) = git::recv::capabilties_and_possibly_refs(&mut self.line_reader, self.version)?;
        self.service = Some(service);
        Ok(client::SetServiceResponse {
            actual_protocol: self.version,
            capabilities,
            refs,
        })
    }

    fn request(
        &mut self,
        _write_mode: client::WriteMode,
        _on_drop: Vec<client::MessageKind>,
        _handle_progress: Option<client::HandleProgress>,
    ) -> Result<client::RequestWriter, client::Error> {
        let service = self.service.expect("handshake() must have been called first");
        let url = append_url(&self.url, service.as_str());
        let headers = &[format!("Content-Type: application/x-git-{}-request", service.as_str())];
        let PostResponse {
            headers,
            body,
            post_body,
        } = self.http.post(&url, headers)?;
        self.line_writer.inner = post_body;
        unimplemented!("http line writer: POST")
    }
}

pub fn connect(url: &str, version: crate::Protocol) -> Result<Transport, Infallible> {
    Ok(Transport::new(url, version))
}
