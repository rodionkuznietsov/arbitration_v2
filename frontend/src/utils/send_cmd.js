// import { API_URL } from "@/config";

export async function send_cmd(
    token,
    event,
    data
) {
    event
    data
    
    try {
        alert(event)
    //     await fetch(`${API_URL}/user/state/update`, {
    //         method: 'POST',
    //         headers: {
    //             'Content-Type': 'application/json',
    //             'Authorization': `Bearer ${token}`
    //         },
    //         body: JSON.stringify({ event, data })
    //     })
    } catch(err) {
        console.log(err)
    }
}