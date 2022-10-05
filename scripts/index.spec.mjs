import path from 'path';
import { fileURLToPath } from 'url';
import * as chai from 'chai';
import chaiAsPromised from 'chai-as-promised';
import { Polly, setupMocha as setupPolly } from '@pollyjs/core';
import FSPersister from '@pollyjs/persister-fs';
import NodeHttpAdapter from '@pollyjs/adapter-node-http';
import { healthCheck } from './index.mjs';

chai.use(chaiAsPromised);
const { assert, expect } = chai;
const __dirname = path.dirname(fileURLToPath(import.meta.url));

Polly.register(NodeHttpAdapter);
Polly.register(FSPersister);

describe('index.js', function () {
    /* eslint-disable mocha/no-setup-in-describe */
    setupPolly({
        adapters: ['node-http'],
        persister: 'fs',
        persisterOptions: {
            fs: {
                recordingsDir: path.resolve(__dirname, './recordings'),
            },
        },
    });
    /* eslint-enable mocha/no-setup-in-describe */

    describe('/health)', function () {
        it('should respond with HTTP 200', async function () {
            const res = await healthCheck();
            assert.strictEqual(res.status, 200);
        });
    });
});
