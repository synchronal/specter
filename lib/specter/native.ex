defmodule Specter.Native do
  @moduledoc false
  use Rustler, otp_app: :specter, crate: :specter_nif

  @opaque t() :: reference()

  @type uuid() :: String.t()

  @spec init() :: {:ok, t()}
  def init, do: error()

  defp error, do: :erlang.nif_error(:nif_not_loaded)
end
