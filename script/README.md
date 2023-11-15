- Start a PostgreSQL Server
- Start Reacher with Bulk Endpoints enabled.
	- e.g `.env` :
	     RCH_ENABLE_BULK=1
            DATABASE_URL="postgresql://user:temporary@localhost"
- Build `script` with `cargo build`
- Inside `script/.env` set `DATABASE_URL` (same as reacher) and `DAYS_OLD` e.g 1,2 etc
- Send a request to the Bulk End point and wait for the job id to be allotted.
- Run the script with `cargo run`
