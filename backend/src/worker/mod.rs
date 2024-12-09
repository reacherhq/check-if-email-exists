// Reacher - Email Verification
// Copyright (C) 2018-2023 Reacher

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// Each file corresponds to one step in the worker pipeline. The worker pipeline
// is defined as:
// - consume from RabbitMQ
// - do the work (i.e. check the email)
// - send response (either to the reply_to queue or save to the database)

pub mod consume;
pub mod do_work;
pub mod single_shot;

pub use consume::{run_worker, setup_rabbit_mq};
