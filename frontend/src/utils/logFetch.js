import { API_URL } from "@/config";

export async function logBotEvent(event, data, token) {
    try {
        const timestamp = Math.floor(Date.now() / 1000);
        await fetch(`${API_URL}/user/bot/add/log`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`
            },
            body: JSON.stringify({ event, data, timestamp })
        })
    } catch(err) {
        console.log('Ошибка при отправке лога', err)
    }
}

export async function getBotLogs(token) {
    try {
        const response = await fetch(`${API_URL}/user/bot/get/logs/`, {
            method: 'GET',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`
            }
        });
        if (!response.ok) {
            console.log("Ошибка сервера:", response.status, response.statusText);
            return null
        }

        const data = await response.json()
        return data.message.logs
    } catch(err) {
        console.log("Ошибка при получении логов")
        return null
    }
}