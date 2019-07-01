const path = require('path');

const MonacoWebpackPlugin = require('monaco-editor-webpack-plugin');
const { ProvidePlugin } = require('webpack');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

module.exports = function override(config) {
    const { output, module, resolve, plugins } = config;

    // https://github.com/facebook/create-react-app/issues/4912#issuecomment-472223885
    module.rules.forEach(rule => {
        (rule.oneOf || []).forEach(oneOf => {
            if (oneOf.loader && oneOf.loader.indexOf('file-loader') >= 0) {
                // Make file-loader ignore WASM files
                oneOf.exclude.push(/\.wasm$/);
            }
        });
    });
    return {
        ...config,
        output: {
            ...output,
            webassemblyModuleFilename: 'static/wasm/[modulehash].wasm',
        },
        resolve: {
            ...resolve,
            extensions: [...resolve.extensions, '.wasm'],
        },
        plugins: [
            new WasmPackPlugin({
                crateDirectory: path.resolve(__dirname, '..', 'naru-wasm'),
                extraArgs: `--out-dir ${path.resolve(__dirname, 'src', 'naru')} --out-name index`,
            }),
            new ProvidePlugin({
                TextDecoder: ['text-encoding', 'TextDecoder'],
                TextEncoder: ['text-encoding', 'TextEncoder'],
            }),
            new MonacoWebpackPlugin({
                languages: ['lua'],
            }),
            ...plugins,
        ],
    };
};
