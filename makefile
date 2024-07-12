app: dargo-server/target/release/dargo-server

prepare:
	cd dargo-client; npm ci

clean:
	rm dargo-client/dist/*
	cd dargo-server; cargo clean

dargo-client/dist/bundle.js dargo-client/dist/bundle.js.map &: dargo-client/src/*
	cd dargo-client; npm run build-prod

dargo-server/target/release/dargo-server: dargo-client/dist/bundle.js dargo-client/dist/bundle.js.map dargo-client/index.html dargo-client/index.css
	cd dargo-server; cargo build --release
