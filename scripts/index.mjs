import axios from 'axios';

const addr = 'http://0.0.0.0:3000';
const client = axios.create({
    timeout: 20 * 1000,
});

export async function healthCheck() {
    const url = new URL(`${addr}/health`);
    return client.get(url);
}

export async function createTask(taskType, startAfterDateTime) {
    const url = new URL(`${addr}/task/create`);
    const body = { task_type: taskType, start_after: startAfterDateTime };
    return client.post(url, body);
}

export async function listTasks() {
    const url = new URL(`${addr}/task/list`);
    return client.get(url);
}
