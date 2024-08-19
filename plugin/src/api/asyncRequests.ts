import { apiUrl } from '../utils/network'

async function handleAsyncApiResponse(response: Response, getterMethod: string): Promise<Response> {
  const pid = await response.text()

  try {
    await waitProcess(pid)

    return await fetch(`${apiUrl}/${getterMethod}/${pid}`, {
      method: 'GET',
      redirect: 'follow',
      headers: {
        'Content-Type': 'application/octet-stream'
      }
    })
  } catch (e) {
    throw new Error(`Error while running process with id ${pid}, error: ${String(e)}`)
  }
}

export async function asyncPost(method: string, getterMethod: string, data: any): Promise<Response> {
  const response = await fetch(`${apiUrl}/${method}`, {
    method: 'POST',
    redirect: 'follow',
    headers: {
      Accept: 'application/json',
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({ ...data })
  })

  return await handleAsyncApiResponse(response, getterMethod)
}

export async function asyncGet(method: string, getterMethod: string): Promise<Response> {
  const response = await fetch(`${apiUrl}/${method}`, {
    method: 'GET',
    redirect: 'follow',
    headers: {
      'Content-Type': 'application/octet-stream'
    }
  })

  return await handleAsyncApiResponse(response, getterMethod)
}

export async function waitProcess(pid: string): Promise<string> {
  while (true) {
    const response = await fetch(`${apiUrl}/process_status/${pid}`, {
      method: 'GET',
      redirect: 'follow',
      headers: {
        'Content-Type': 'application/octet-stream'
      }
    })

    if (!response.ok) {
      throw new Error(`Error while running process with id ${pid}, error: ${response.statusText}`)
    }

    const status = await response.text()

    switch (status.at(0)) {
      case 'C':
        return status
      case 'E':
        throw new Error(`Error while running process with id ${pid}, error: ${status}`)
      default:
        break
    }

    await new Promise((resolve) => setTimeout(resolve, 1000))
  }
}
