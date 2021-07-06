module.exports = {
  'env': {
    'browser': true,
    'es2021': true,
  },
  'extends': [
    'plugin:react/recommended',
    'google',
  ],
  'parser': '@typescript-eslint/parser',
  'parserOptions': {
    'ecmaFeatures': {
      'jsx': true,
    },
    'ecmaVersion': 12,
    'sourceType': 'module',
  },
  'plugins': [
    'react',
    '@typescript-eslint',
  ],
  'rules': {
    'object-curly-spacing': 'off',
    'require-jsdoc': 'off',
    'max-len': [2, 500, 4],
    'no-unused-vars': 'off',
    '@typescript-eslint/no-unused-vars': 'error',
  },
};
