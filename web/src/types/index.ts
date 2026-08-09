export interface ModuleDto {
  id: string
  name: string
  version: string
}

export interface CommandDto {
  name: string
  description: string
}

export interface CoreServiceDto {
  name: string
  status: string
}

export interface ExecuteCommandRequest {
  name: string
  args: string[]
}

export interface ExecuteCommandResponse {
  output: string
}