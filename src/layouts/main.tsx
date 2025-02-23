import { Outlet } from 'react-router'

export function MainLayout() {
  return (
    <>
      <div data-tauri-drag-region className="bg-woodsmoke-900 h-11"></div>

      <div className="flex h-full flex-1">
        {/* .sidebar */}
        <main className="flex flex-1 px-4">
          <Outlet />
        </main>
      </div>
    </>
  )
}
