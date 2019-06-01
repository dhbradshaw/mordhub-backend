# MordHub

## Building and Running

### Docker

This is the recommended method if you're not doing any development on the backend.

First, get a steam key from [here](https://steamcommunity.com/dev/apikey).

Create a file in the project root named `.env`, and put the following contents:
```
POSTGRES_USER=admin
POSTGRES_PASSWORD=Password1
DATABASE_URL=postgres://admin:Password1@postgres/mordhub
STEAM_API_KEY=YOURKEYHERE
COOKIE_SECRET=RMCLVcuHYboeeOgDxm53aLkNPKC4UMWU
SITE_URL=http://localhost:3000
```
Make sure to put your steam API key instead of `YOURKEYHERE`. Note the differences which make this `.env` file incompatible with the manual method shown below.

Next, install [`docker`](https://docs.docker.com/install/) and [`docker-compose`](https://docs.docker.com/compose/install/). Then, simply run:
```
docker-compose up
```
from the project root. It may take about 10 minutes to compile from scratch, and uses about ~3GB of disk space at the time of writing (1.7GB for the rust install, 1.2GB for the dependencies and build artifacts of the project itself).

Also make sure to stop any local postgres services you might be running as docker will complain about port 5432 being used.

There's also a pre-built image at [`sixtyfive/mordhub-backend`](https://hub.docker.com/r/sixtyfive/mordhub-backend) if you'd rather download things than build yourself.

### Manual

1. Install PostgreSQL 11 for your platform. Follow the guide [here](https://github.com/diesel-rs/diesel/blob/master/guide_drafts/backend_installation.md). On Windows, I used the enterprise installer then added the `bin/` folder to my `Path` environment variable. You don't need to bother installing the `stack builder` component but install the others, if prompted. Remember the password you set for the superuser account during installation. On Linux, you will need to [set the password manually](https://serverfault.com/a/248162). Leave the default port (5432).

2. Install `rustup`, from [here](https://rustup.rs/). Should be pretty straightforward. Gives you access to `cargo`, which is the Rust package manager and build system.

3. Install `diesel_cli` using the command: `cargo install diesel_cli --no-default-features --features postgres`. Should take 5 min or so. You can get on with steps 7, 8, 9 in the meantime.

4. (Windows only) You now need to install OpenSSL. This is really, really horrible and I apologise in advance (thank Microsoft for me). I'm roughly paraphrasing the instructions [here](https://docs.rs/crate/openssl/0.10.7) now: go [here](http://slproweb.com/products/Win32OpenSSL.html) and select the `Win64 OpenSSL v1.1.1b MSI (experimental)` option. Remember where you installed it. Write `set OPENSSL_DIR=C:\Program Files\OpenSSL-Win64` or wherever you installed it, so that `rust-openssl` knows where to find it.

5. Compile `mordhub`. Inside the git repo, run `cargo build`. Should take another 5 min.

6. (optional) Install `cargo-watch` with: `cargo install cargo-watch`. Should take 2 min. This allows for the server to reload itself when you change a file (you still need to refresh the browser, though).

7. Get a steam API key from [here](https://steamcommunity.com/dev/apikey). Set the domain as something like `http://localhost:3000` (it doesn't really matter).

8. Create a database for `mordhub` to use. Go to the command line and type `psql -U postgres` and enter the password from earlier. A different command prompt should open. Type `CREATE DATABASE mordhub;`. Type `\q` to exit after that. 

9. Create a file in the project root called `.env`. Put the following contents:
```
DATABASE_URL=postgres://postgres:DBPASSWORD@localhost/mordhub
STEAM_API_KEY=STEAMAPIKEYHERE
COOKIE_SECRET=Ur6FvHby2XJ8THRNdnUD8bFaS6GFsw2p
SITE_URL=http://localhost:3000
```
Replace `DBPASSWORD` and `STEAMAPIKEYHERE` as appropriate. The cookie secret doesn't matter much if you're just testing the server, but in production it should be a totally random 32-byte string. On Linux, if you created a user with a different username to `postgres`, then edit the `DATABASE_URL` accordingly (e.g `postgres://myuser:mypass@localhost/mordhub`).

10. Finish database creation with `diesel setup` inside the project root.

11. Finally, you're done! Run the project with `cargo run` and open `http://localhost:3000` in your browser. If you installed `cargo-watch`, you can instead use `cargo watch -x run` to automatically re-run the server when you edit a file (this is required for most files, especially templates, as they are compiled during program startup).

12. Celebrate with a glass of champagne.

## TODO
- [ ] Switch to Askama templating engine
- [ ] Pagination
- [x] Image upload
- [ ] Clean up routes/auth.rs and models/loadout.rs
- [ ] Comp scene tracker
- [ ] Mod ideas page
- [x] Get license from KickOff
- [ ] Color thief (?)
- [ ] Native loadout importer
- [ ] Better 'missing image' icon
- [ ] Loadout categories
- [ ] Search function
- [ ] Team loadouts
