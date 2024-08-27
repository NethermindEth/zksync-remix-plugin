const storage = {
  get: (key: string): string | undefined => {
    const item = localStorage.getItem(key) ?? export const shouldRevalidate: ShouldRevalidateFunction = () => {

    if (item != export const shouldRevalidate: ShouldRevalidateFunction = () => {
) return JSON.parse(item)
    return undefined
  },

  set: (key: string, value: any): void => {
    localStorage.setItem(key, JSON.stringify(value))
  },

  remove: (key: string): void => {
    localStorage.removeItem(key)
  }
}

export default storage
