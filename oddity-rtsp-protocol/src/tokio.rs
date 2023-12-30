use tokio_util::codec::{Decoder, Encoder};

use bytes::BytesMut;

use super::{
    error::Error,
    interleaved::{self, InterleavedParser, MaybeInterleaved},
    io::Target,
    parse::{Parser, Status},
    serialize::Serialize,
};

pub struct Codec<T: Target> {
    state: State,
    parser: Parser<T::Inbound>,
    interleaved_parser: InterleavedParser,
}

enum State {
    Init,
    ParseMessage,
    ParseInterleaved,
}

impl<T: Target> Codec<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: State::Init,
            parser: Parser::<T::Inbound>::new(),
            interleaved_parser: InterleavedParser::new(),
        }
    }
}

impl<T: Target> Default for Codec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Target> Decoder for Codec<T> {
    type Item = MaybeInterleaved<T::Inbound>;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if matches!(self.state, State::Init) {
            if src.is_empty() {
                return Ok(None);
            } else if src[0] == interleaved::MAGIC {
                self.state = State::ParseInterleaved;
            } else {
                self.state = State::ParseMessage;
            }
        };

        match &mut self.state {
            State::Init => unreachable!(),
            State::ParseMessage => match self.parser.parse(src)? {
                Status::Done => {
                    self.state = State::Init;
                    let parser = std::mem::replace(&mut self.parser, Parser::<T::Inbound>::new());
                    Ok(Some(
                        parser
                            .into_message()
                            .map(MaybeInterleaved::<T::Inbound>::Message)?,
                    ))
                }
                Status::Hungry => Ok(None),
            },
            State::ParseInterleaved => match self.interleaved_parser.parse(src) {
                Some(parsed) => {
                    let (channel, payload) = parsed?;
                    self.state = State::Init;
                    self.interleaved_parser = InterleavedParser::new();
                    Ok(Some(MaybeInterleaved::<T::Inbound>::Interleaved {
                        channel,
                        payload,
                    }))
                }
                None => Ok(None),
            },
        }
    }
}

impl<T: Target> Encoder<MaybeInterleaved<T::Outbound>> for Codec<T> {
    type Error = Error;

    fn encode(
        &mut self,
        item: MaybeInterleaved<T::Outbound>,
        dst: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        item.serialize(dst)
    }
}
