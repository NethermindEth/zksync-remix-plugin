import React from 'react'
import './backgroundNotices.css'
import { PLUGIN_INFO_CONTENT_ARRAY } from '@/utils/constants'

const BackgroundNotices = () => {
  return (
    <div className={'bg-transparent'}>
      {
        <ul className="list-group">
          {PLUGIN_INFO_CONTENT_ARRAY.map((notice, index) => {
            return (
              <li
                className="list-group-item d-flex justify-content-left align-items-center bg-notices-text bg-primary"
                key={index}
              >
                {notice}
              </li>
            )
          })}
        </ul>
      }
    </div>
  )
}

export default BackgroundNotices
