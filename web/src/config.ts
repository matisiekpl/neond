export const Config = {
    serverUrl: window.location.href.includes('localhost')
        ? 'http://localhost:3000'
        : `${window.location.protocol}//${window.location.hostname}${window.location.port ? `:${window.location.port}` : ''}`,
}