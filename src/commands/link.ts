import { Interaction } from 'discord.js';
import { SlashCommandBuilder } from '@discordjs/builders';

import { prismaClient } from '../shared/prismaClient';

const commandData = new SlashCommandBuilder()
	.setName('link')
	.setDescription('Links an user to an address.')
	.addStringOption((option) =>
		option.setName('address').setDescription('Wallet address input')
	);

async function execute(interaction: Interaction) {
	if (interaction.isCommand()) {
		const address = interaction.options.getString('address');
		const { id } = interaction.user;

		console.log(address);

		if (address) {
			try {
				const user = await prismaClient.user.create({
					data: {
						discord_id: id,
						wallet: address,
					}
				});

				console.log(user);
			}
			catch (err) {
				console.log(err);
			}
		}

		return interaction.reply(`Pupupu! Ethereum Address: ${address} has been linked to your account!`);
	}
}

export { commandData, execute };