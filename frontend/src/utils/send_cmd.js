import { API_URL } from "@/config";

export async function send_cmd(
    token,
    event,
    market_type,
    value
) {
    if (event.toLowerCase() == "exchange_update") {
        try {
            const response = await fetch(`${API_URL}/user/state/update`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${token}`
                },
                body: JSON.stringify({ 
                    event: event, 
                    data: {
                        exchange_name: value.toLowerCase(),
                        market_type: market_type.toLowerCase()
                    } 
                })
            })

            if (!response.ok) {
                const errorData = await response.json()
                alert(JSON.stringify(errorData))
            }
        } catch(err) {
            alert(JSON.stringify(err))
        }
    }
}