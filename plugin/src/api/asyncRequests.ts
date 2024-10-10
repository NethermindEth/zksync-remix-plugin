import { CompiledArtifact, ContractFile } from '@/types/contracts'
import { GeneratePresignedUrlsRequest, GeneratePresignedUrlsResponse } from '@/api/types'

export const GENERATE_LAMBDA_URL = 'https://7462iuvrevrwndflwr5r6nf2340owkmz.lambda-url.ap-southeast-2.on.aws/'
export const COMPILE_LAMBDA_URL = 'https://w6myokcnql4lw2oel27xj52njy0cfrto.lambda-url.ap-southeast-2.on.aws/'
export const VERIFY_LAMBDA_URL = '' // TODO: proper url
export const POLL_LAMBDA_URL = 'https://a2pwosrlela2fwuz5tsdznkgma0ovkuj.lambda-url.ap-southeast-2.on.aws/'

export async function asyncPost<T>(methodUrl: string, getterMethodUrl: string, data: any, pid: string): Promise<T> {
  const response = await fetch(methodUrl, {
    method: 'POST',
    redirect: 'follow',
    headers: {
      Accept: 'application/json',
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({ ...data })
  })

  if (!response.ok) {
    const text = await response.text()
    throw new Error(`Calling ${methodUrl} failed with ${text}`)
  }

  return await waitProcess<T>(getterMethodUrl, pid)
}

async function post<T>(methodUrl: string, data: any): Promise<T> {
  const response = await fetch(methodUrl, {
    method: 'POST',
    redirect: 'follow',
    headers: {
      Accept: 'application/json',
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({ ...data })
  })

  const text = await response.text()
  if (!response.ok) {
    throw new Error(`Calling ${methodUrl} failed with ${text}`)
  }

  return JSON.parse(text) as T
}

export async function get(getterMethodUrl: string): Promise<Response> {
  const response = await fetch(`${getterMethodUrl}`, {
    method: 'GET',
    redirect: 'follow',
    headers: {
      'Content-Type': 'application/octet-stream'
    }
  })

  return response
}

export async function waitProcess<T>(getterMethodUrl: string, pid: string): Promise<T> {
  const OK = 200
  const ACCEPTED = 202

  while (true) {
    const response = await fetch(`${getterMethodUrl}/${pid}`, {
      method: 'GET',
      redirect: 'follow',
      headers: {
        'Content-Type': 'application/json'
      }
    })

    switch (response.status) {
      case OK: {
        const text = await response.text()
        return JSON.parse(text) as T
      }
      case ACCEPTED:
        break
      default: {
        const text = await response.text()
        throw new Error(`Error while running process with id ${pid}, error: ${text}`)
      }
    }

    await new Promise((resolve) => setTimeout(resolve, 1000))
  }
}

// Return ID associated with the task
export async function initializeTask(files: ContractFile[]): Promise<string> {
  const request: GeneratePresignedUrlsRequest = {
    files: files.map((el) => el.file_content)
  }

  const response = await post<GeneratePresignedUrlsResponse>(GENERATE_LAMBDA_URL, request)
  const expected = request.files.length
  const actual = response.presigned_urls.length
  if (actual !== expected) {
    throw new Error(`Expected number of URLs: ${expected}, actual: ${actual}`)
  }

  const uploadTasks = response.presigned_urls.map((url, i) => uploadFileToS3(url, request.files[i]))
  await Promise.all(uploadTasks)

  return response.id
}

async function uploadFileToS3(presignedUrl: string, file: string) {
  const blob = new Blob([file], { type: 'text/plain' })

  const uploadResponse = await fetch(presignedUrl, {
    method: 'PUT',
    body: blob,
    headers: {
      'Content-Type': 'text/plain'
    }
  })

  if (!uploadResponse.ok) {
    const text = await uploadResponse.text()
    throw new Error(`Failed to upload file: ${text ?? uploadResponse.statusText}`)
  }
}

export async function downloadArtifacts(presignedUrls: string[]): Promise<CompiledArtifact[]> {
  const promises = presignedUrls.map((el) => get(el))
  const responses = await Promise.all(promises)

  // TODO: cleanup
  {
    responses.forEach((el) => console.log('downloadArtifacts::response:', el))
  }

  const textPromises = responses.map((el) => el.text())
  const files = await Promise.all(textPromises)

  return files.map((file): CompiledArtifact => {
    console.log(file)
    return {
      file_path: 'huh',
      file_content: file,
      is_contract: true
    }
  })
}
