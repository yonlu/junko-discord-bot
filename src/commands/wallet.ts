import { Interaction } from 'discord.js';
import { SlashCommandBuilder } from '@discordjs/builders';
import axios from 'axios';

import { getBcoinBalance, getNativeBalance } from '../contracts/bcoinContract';
import { prismaClient } from '../shared/prismaClient';

const commandData = new SlashCommandBuilder()
  .setName('wallet')
  .setDescription('Replies with selected address BNB value.');

async function execute(interaction: Interaction) {
  if (interaction.isCommand()) {
    const { data } = await axios.get(
      'https://api.coingecko.com/api/v3/coins/bomber-coin'
    );

    const { usd } = data.market_data.current_price;

    const { id } = interaction.user;

    const user = await prismaClient.user.findFirst({
      where: {
        discord_id: id
      }
    });

    if (user?.wallet) {
      const walletBalance = await getNativeBalance(user?.wallet);
      const bcoinBalance = await getBcoinBalance(user?.wallet);
      const bcoinInUSD = new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' })
        .format((Number(bcoinBalance) * usd));

      return interaction.reply(
        'Balance:\n' +
        `${walletBalance} BNB\n` +
        `${bcoinBalance} BCOIN (${bcoinInUSD})`
      );
    }
  }
}

export { commandData, execute };