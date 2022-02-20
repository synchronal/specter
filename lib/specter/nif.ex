defmodule Specter.NIF do
  use Rustler, otp_app: :specter, crate: :specter_nif

  def new, do: :erlang.nif_error(:nif_not_loaded)
  def get(_ref), do: :erlang.nif_error(:nif_not_loaded)
  def set(_ref, _string), do: :erlang.nif_error(:nif_not_loaded)
end
