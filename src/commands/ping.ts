import { SlashCommandBuilder } from '@discordjs/builders';
import { IInteraction } from '../types';

module.exports = {
  data: new SlashCommandBuilder()
    .setName('ping')
    .setDescription('Replies with Xanadu!'),
  async execute(interaction: IInteraction) {
    await interaction.reply('Xanadu!');
  },
};
