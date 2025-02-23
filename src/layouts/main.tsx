import { Outlet } from 'react-router'

export function MainLayout() {
  return (
    <>
      {/* <Titlebar /> */}

      <div className="flex-1 flex h-full">
        {/* <Sidebar /> */}
        <main className="flex-1 flex px-4">
          <Outlet />
        </main>
      </div>
    </>
  )
}
