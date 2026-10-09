// Each file corresponds to one step in the worker pipeline. The worker pipeline
// is defined as:
// - consume from RabbitMQ
// - do the work (i.e. check the email)
// - send response (either to the reply_to queue or save to the database)

pub mod consume;
pub mod do_work;
pub mod single_shot;

pub use consume::{run_worker, setup_rabbit_mq};
