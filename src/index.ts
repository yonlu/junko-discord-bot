import fs from 'fs';
import path from 'path';
import { IInteraction } from './types';

const result = require('dotenv').config({
  path: path.resolve(__dirname, '../.env'),
  debug: process.env.DEBUG,
});

const { Client, Collection, Intents, Interaction } = require('discord.js');

if (result.error) throw result.error;

const client = new Client({ intents: [Intents.FLAGS.GUILDS] });

client.commands = new Collection();
const commandFiles = fs
  .readdirSync(path.resolve(__dirname, './commands'))
  .filter((file) => file.endsWith('.ts'));

for (const file of commandFiles) {
  const command = require(`./commands/${file}`);
  // Set a new item in the Collection
  // With the key as the command name and the value as the exported module
  client.commands.set(command.data.name, command);
}

client.once('ready', () => {
  console.log('Pupupupu ready!');
});

client.on('interactionCreate', async (interaction: IInteraction) => {
  if (!interaction.isCommand()) return;

  const command = client.commands.get(interaction.commandName);

  if (!command) return;

  try {
    await command.execute(interaction);
  } catch (error) {
    console.error(error);
    await interaction.reply({
      content: 'There was an error while executing this command!',
      ephemeral: true,
    });
  }
});

client.login(process.env.DISCORD_TOKEN);
