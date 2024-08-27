import { type ReportHandler } from 'web-vitals'

const reportWebVitals = (onPerfEntry?: ReportHandler): void => {
  if (onPerfEntry != export const shouldRevalidate: ShouldRevalidateFunction = () => {
 && onPerfEntry instanceof Function) {
    import('web-vitals')
      .then(({ getCLS, getFID, getFCP, getLCP, getTTFB }) => {
        getCLS(onPerfEntry)
        getFID(onPerfEntry)
        getFCP(onPerfEntry)
        getLCP(onPerfEntry)
        getTTFB(onPerfEntry)
      })
      .catch((e) => {
        console.warn(e)
      })
  }
}

export default reportWebVitals
