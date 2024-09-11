import { Actor, HttpAgent } from '@dfinity/agent';
import { AuthClient } from '@dfinity/auth-client';
import {
	example_backend,
	idlFactory as exampleBackendIdlFactory
} from 'declarations/example_backend';
import { idlFactory as icSignerIdlFactory } from 'declarations/signer';
import { html, render } from 'lit-html';
import logo from './logo2.svg';

class App {
	greeting = '';

	constructor() {
		this.#render();
	}

	#handleLogin = async (e) => {
		e.preventDefault();

		console.log('Handle login');

		// create an auth client
		let authClient = await AuthClient.create();

		// start the login process and wait for it to finish
		await new Promise((resolve) => {
			authClient.login({
				identityProvider: 'http://avqkn-guaaa-aaaaa-qaaea-cai.localhost:4943/',
				onSuccess: resolve
			});
		});

		// At this point we're authenticated, and we can get the identity from the auth client:
		const identity = authClient.getIdentity();
		console.log({ identity });
		// Using the identity obtained from the auth client, we can create an agent to interact with the IC.
		const agent = new HttpAgent({ identity });
		console.log({ agent });
		// Fetch root key. TODO: Do this only for local and other whitelisted non-mainnet networks.
		// TODO: Pass the network and canister IDs as arguments, do NOT compile them in.
		agent.fetchRootKey().catch((err) => {
			console.warn('Unable to fetch root key. Check to ensure that your local replica is running');
			console.error(err);
		});

		// Using the interface description of our webapp, we create an actor that we use to call the service methods.
		const actor = Actor.createActor(exampleBackendIdlFactory, {
			agent,
			canisterId: 'bw4dl-smaaa-aaaaa-qaacq-cai'
		});
		console.log({ actor });
		actor.greet('Alice').then((greeting) => {
			console.log({ greeting });
		});
		const ic_signer = Actor.createActor(icSignerIdlFactory, {
			agent,
			canisterId: 'asrmz-lmaaa-aaaaa-qaaeq-cai'
		});
		console.log({ ic_signer });
		ic_signer.sign('Signer').then((greeting) => {
			console.log({ greeting });
		});

		return false;
	};
	#handleSubmit = async (e) => {
		e.preventDefault();
		const name = document.getElementById('name').value;
		this.greeting = await example_backend.greet(name);
		this.#render();
	};

	#render() {
		let body = html`
			<main>
				<img src="${logo}" alt="DFINITY logo" />
				<br />
				<br />
				<button id="login">Login 5!</button>
				<form id="hi" action="#">
					<label for="name">Enter your name: &nbsp;</label>
					<input id="name" alt="Name" type="text" />
					<button type="submit">Click Me!</button>
				</form>
				<section id="greeting">${this.greeting}</section>
			</main>
		`;
		render(body, document.getElementById('root'));
		document.querySelector('#login').addEventListener('click', this.#handleLogin);
		document.querySelector('#hi').addEventListener('submit', this.#handleSubmit);
	}
}

export default App;
