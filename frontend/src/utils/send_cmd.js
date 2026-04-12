// import { API_URL } from "@/config";

import { API_URL } from "@/config"

export async function send_cmd(
    token,
    event,
    data
) {
    event
    data
    
    try {
        const response = await fetch(`${API_URL}/user/state/update`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`
            },
            body: JSON.stringify({ event, data })
        })

        if (!response.ok) {
            const errorData = await response.json()
            alert(JSON.stringify(errorData))
        }
    } catch(err) {
        console.log(err)
    }
}