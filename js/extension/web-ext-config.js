require('dotenv/config');

module.exports = {
  sourceDir: 'dist/prod',
  artifactsDir: 'artifacts',
  sign: {
    apiKey: process.env.WEB_EXT_API_KEY || '',
    apiSecret: process.env.WEB_EXT_API_SECRET || '',
  },
};
