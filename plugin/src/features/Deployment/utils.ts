import { RemixClient } from '@/PluginClient'
import { TaskFailure } from '@/api/types'

export async function handleCompilationFailure(remixClient: RemixClient, taskFailure: TaskFailure) {
  await remixClient.terminal.log({
    value: taskFailure.message,
    type: 'error'
  })

  const errorLets = taskFailure.message.trim().split('\n')

  // remove last element if it's starts with `Error:`
  if (errorLets[errorLets.length - 1].startsWith('Error:')) {
    errorLets.pop()
  }

  // break the errorLets in array of arrays with first element contains the string `Plugin diagnostic`
  const errorLetsArray = errorLets.reduce(
    (acc: any, curr: any) => {
      if (curr.startsWith('error:') || curr.startsWith('warning:')) {
        acc.push([curr])
      } else {
        acc[acc.length - 1].push(curr)
      }
      return acc
    },
    [['errors diagnostic:']]
  )

  // remove the first array
  errorLetsArray.shift()

  // eslint-disable-next-line @typescript-eslint/no-misused-promises
  const promises = errorLetsArray.map(async (errorLet: any) => {
    const errorType = errorLet[0].split(':')[0].trim()
    const errorTitle = errorLet[0].split(':').slice(1).join(':').trim()
    const errorLine = errorLet[1].split(':')[1].trim()
    const errorColumn = errorLet[1].split(':')[2].trim()
    // join the rest of the array
    const errorMsg = errorLet.slice(2).join('\n')

    await remixClient.editor.addAnnotation({
      row: Number(errorLine) - 1,
      column: Number(errorColumn) - 1,
      text: errorMsg + '\n' + errorTitle,
      type: errorType
    })
  })

  await Promise.all(promises)

  // trim sierra message to get last line
  const lastLine = taskFailure.message.trim().split('\n').pop()?.trim()

  remixClient.emit('statusChanged', {
    key: 'failed',
    type: 'error',
    title: (lastLine ?? '').startsWith('Error') ? lastLine : 'Verification Failed'
  })
}
