import { Interaction } from 'discord.js';
import { SlashCommandBuilder } from '@discordjs/builders';
import axios from 'axios';

const commandData = new SlashCommandBuilder()
  .setName('coin')
  .setDescription('Replies with selected token price.')
  .addStringOption((option) =>
    option.setName('input').setDescription('Coin name input')
  );

async function execute(interaction: Interaction) {
  if (interaction.isCommand()) {
    const coinName = interaction.options.getString('input');

    const { data } = await axios.get(
      `https://api.coingecko.com/api/v3/coins/${coinName}`
    );

    const { name, symbol } = data;
    const { usd } = data.market_data.current_price;
    const { price_change_percentage_24h, price_change_percentage_7d } = data.market_data;

    const formattedPercentageChange = price_change_percentage_24h.toFixed(2);
    const formattedPercentageChange7d = price_change_percentage_7d.toFixed(2);

    const parsedPercentageChangeString = price_change_percentage_24h >= 0 ?
      `${formattedPercentageChange}% 📈`
      : `${formattedPercentageChange}% 📉`;
    const parsedPercentageChange7dString = price_change_percentage_7d >= 0 ?
      `${formattedPercentageChange7d}% 📈`
      : `${formattedPercentageChange7d}% 📉`;

    return interaction.reply(
      `${symbol.toString().toUpperCase()} (${name}), Current price: $${usd}\n` +
      `Price change 24 hours: ${parsedPercentageChangeString}\n` +
      `Price change 7 days: ${parsedPercentageChange7dString}`
    );
  }
}

export { commandData, execute };