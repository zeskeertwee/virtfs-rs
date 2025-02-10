import Ora from "ora";
import Loki from "lokijs";
import Snoowrap from "snoowrap";

export default class Bot {
	// Config objects
	#snoowrapConfig; #lokiConfig;
	// Context variables
	#client; #spinner; #db;
	// Polling variables
	#pollInterval; #pollRate;

	constructor(config = {}) {
		// Sanitize inputs
		this.#pollRate = Math.max(5_000, 1_000 * (60 / (config.rpm || 10_000)));

		// Spinner Init
		this.#spinner = new Ora({ spinner: 'earth' });
		this.#spinner.start('Firing up Reddit bot');

		// Snoowrap config
		this.#snoowrapConfig = config.snoowrap;

		// Save database config
		this.#lokiConfig = config.loki;

		try {
			this.#client = new Snoowrap(this.#snoowrapConfig);

			this.#spinner.succeed('Successfully started bot')
		} catch (error) {
			this.#spinner.fail(`[BOT_ERROR] Unable to start bot: ${error.message}`);

			return
		};
	}

	async start() {
		this.#spinner.spinner = 'arc';
		this.#spinner.start('[LOKI_DB] Initializing database');

		// load database
		let loadDBPromise = new Promise(res => {
			this.#db = new Loki(this.#lokiConfig.path, {
				autoload: this.#lokiConfig.autoload,
				autosave: this.#lokiConfig.autosave,
				autosaveInterval: this.#lokiConfig.autosaveInterval,
				autoloadCallback: () => { res() }
			});
		})

		// Wait for the DataBase to initialize
		await loadDBPromise;

		let replies = this.#db.getCollection('replies');
		if (replies === null) {
			this.#spinner.spinner = 'grenade';
			this.#spinner.start('[LOKI_DB] Adding users Collection')

			replies = this.#db.addCollection('replies', { unique: ['contentId'] });
		};

		// Inform of success
		this.#spinner.succeed('[LOKI_DB] Successfully initialized database');

		// Check for mentions and messages
		this.#spinner.spinner = 'earth';
		this.#spinner.start(`Attempting to read inbox (messages and mentions): ${Math.round(1_000_000 / this.#pollRate) / 1_000}Hz`);

		// Start polling for updates
		this.#pollInterval = setInterval(() => {
			this.poll(replies).catch(err => {
				clearInterval(this.#pollInterval);
				this.#spinner.fail(`[BOT_POLL_ERROR]: ${err}`);
			})
		}, this.#pollRate)
	}

	async poll(replies) {
		let mentions = await this.#client.getInbox({ filter: 'unread' });

		// Handle mentions
		for (let i = 0; i < mentions.length; i++) {
			const mention = mentions[i], tag = `u/${this.#snoowrapConfig.username}`;

			// Only replies to comments where mentioned
			if (!mention.was_comment && mention.body.includes(tag)) continue;

			// Search for already replied to comments
			const finds = replies.find({ contentId: mention.id });
			if (finds.length > 0) continue;

			// reply to mention and update database
			this.reply(mention);

			// Update the database after every reply
			replies.insert({ contentId: mention.id });
			this.#db.save();
		};
	}

	reply(mention) {
		this.#spinner.spinner = 'earth';
		this.#spinner.info(`[BOT] Replying to mention (${mention.id}) from (${mention.author.name})`)
	}
}
