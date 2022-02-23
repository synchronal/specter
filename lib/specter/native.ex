defmodule Specter.Native do
  @moduledoc false
  use Rustler, otp_app: :specter, crate: :specter_nif

  @type t() :: reference()

  @spec init() :: {:ok, t()}
  def init(args \\ []) do
    args = Keyword.put_new(args, :ice_servers, ["stun:stun.l.google.com:19302"])
    __init__(Enum.into(args, %{}))
  end

  @doc false
  def __init__(_args), do: error()

  defp error, do: :erlang.nif_error(:nif_not_loaded)
end
