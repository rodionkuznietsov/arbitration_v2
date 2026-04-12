import { API_URL } from "@/config";

export async function send_cmd(
    token,
    event,
    data
) {
    try {
        await fetch(`${API_URL}/user/state/update`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`
            },
            body: JSON.stringify({ event, data })
        })
    } catch(err) {
        console.log('Ошибка при отправке команды', err)
    }
}