export async function logBotEvent(event, data) {
    try {
        const timestamp = Math.floor(Date.now() / 1000);
        await fetch('https://unfarming-untethered-flynn.ngrok-free.dev/api/user/bot/log', {
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