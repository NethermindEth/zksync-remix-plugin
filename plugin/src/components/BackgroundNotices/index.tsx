import React from 'react'
import './backgroundNotices.css'

const Notices = [
  'The zkSync Remix Plugin is in Alpha',
  'Solidity contracts are compiled on a server hosted by Nethermind'
]

const BackgroundNotices: React.FC = () => {
  return (
    <div className={'bg-transparent'}>
      <p className='text-center text-md bg-notices-text'>Notices</p>
      {
        <ul className='list-group'>
          {Notices.map((notice, index) => {
            return (
              <li className='list-group-item d-flex justify-content-left align-items-center bg-notices-text bg-primary'
                  key={index}>
                <span className='badge badge-information badge-pilled mr-2'>
                  {index + 1}
                </span>
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
