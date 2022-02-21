defmodule Specter.Native do
  @moduledoc false
  use Rustler, otp_app: :specter, crate: :specter_nif

  @spec init() :: {:ok, reference()}
  def init, do: :erlang.nif_error(:nif_not_loaded)
end
