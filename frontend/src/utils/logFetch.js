export const API_URL = "https://unfarming-untethered-flynn.ngrok-free.dev/api/user/bot"

export async function logBotEvent(event, data) {
    try {
        const timestamp = Math.floor(Date.now() / 1000);
        await fetch(`${API_URL}/add/log`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ event, data, timestamp })
        })
    } catch(err) {
        console.log('Ошибка при отправке лога', err)
    }
}

export async function getBotLogs(tg_user_id) {
    try {
        const response = await fetch(`${API_URL}/get/logs/tg_user_id=${tg_user_id}`);
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