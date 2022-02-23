defmodule Specter.MixProject do
  use Mix.Project

  def project do
    [
      app: :specter,
      deps: deps(),
      elixir: "~> 1.13",
      start_permanent: Mix.env() == :prod,
      version: "0.1.0"
    ]
  end

  def application do
    [
      extra_applications: [:logger]
    ]
  end

  defp deps do
    [
      {:credo, "~> 1.6", only: [:dev, :test], runtime: false},
      {:dialyxir, "~> 1.1", only: [:dev, :test], runtime: false},
      {:ex_doc, "~> 0.28", only: [:dev], runtime: false},
      {:rustler, "~> 0.23"}
    ]
  end
end
