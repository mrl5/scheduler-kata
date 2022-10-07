import path from 'path';
import { fileURLToPath } from 'url';
import * as chai from 'chai';
import chaiAsPromised from 'chai-as-promised';
import { Polly, setupMocha as setupPolly } from '@pollyjs/core';
import FSPersister from '@pollyjs/persister-fs';
import NodeHttpAdapter from '@pollyjs/adapter-node-http';
import { healthCheck, createTask } from './index.mjs';

chai.use(chaiAsPromised);
const { assert, expect } = chai;
const __dirname = path.dirname(fileURLToPath(import.meta.url));

Polly.register(NodeHttpAdapter);
Polly.register(FSPersister);

const oneMinute = 60 * 1000;

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

    describe('/health)', function () {
        it('should respond with HTTP 200', async function () {
            const res = await healthCheck();
            assert.strictEqual(res.status, 200);
        });
    });

    describe('/task)', function () {
        [
            ['TypeA', new Date()],
            ['TypeB', new Date()],
            ['TypeC', new Date(new Date().getTime() + oneMinute)],
        ].forEach((testCase) => {
            it(`it should create new task - ${testCase[0]}`, async function () {
                const [type, datetime] = testCase;
                const { status, data } = await createTask(type, datetime.toISOString());

                assert.strictEqual(status, 202);
                expect(data.task_id).to.be.a('string').and.length.above(0);
            });
        });
    });
});
