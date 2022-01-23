const { SlashCommandBuilder } = require('@discordjs/builders');
const axios = require('axios').default;

module.exports = {
  data: new SlashCommandBuilder()
    .setName('bcoin')
    .setDescription('Replies with BCOIN token price.'),
  async execute(interaction) {
    const { data } = await axios.get(
      'https://api.coingecko.com/api/v3/coins/bomber-coin'
    );

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
      `Current price: $${usd}\n` +
      `Price change 24 hours: ${parsedPercentageChangeString}\n` +
      `Price change 7 days: ${parsedPercentageChange7dString}`
    );
  },
};
