use orchard::vote::Ballot;
use tendermint_abci::Application;
use tendermint_proto::abci::{
    RequestFinalizeBlock, ResponseCommit, ResponseFinalizeBlock,
};

#[derive(Clone, Default)]
pub struct VoteChain;

impl Application for VoteChain {
    fn echo(
        &self,
        request: tendermint_proto::abci::RequestEcho,
    ) -> tendermint_proto::abci::ResponseEcho {
        tendermint_proto::abci::ResponseEcho {
            message: request.message,
        }
    }

    fn info(
        &self,
        _request: tendermint_proto::abci::RequestInfo,
    ) -> tendermint_proto::abci::ResponseInfo {
        Default::default()
    }

    fn init_chain(
        &self,
        _request: tendermint_proto::abci::RequestInitChain,
    ) -> tendermint_proto::abci::ResponseInitChain {
        Default::default()
    }

    fn query(
        &self,
        _request: tendermint_proto::abci::RequestQuery,
    ) -> tendermint_proto::abci::ResponseQuery {
        Default::default()
    }

    fn finalize_block(&self, request: RequestFinalizeBlock) -> ResponseFinalizeBlock {
        println!("finalize_block");
        for tx in request.txs.iter() {
            let ballot: Ballot = bincode::deserialize(&tx).unwrap();
            println!("{}", hex::encode(&ballot.data.domain));
        }
        Default::default()
    }

    fn commit(&self) -> ResponseCommit {
        println!("commit");
        Default::default()
    }
}
