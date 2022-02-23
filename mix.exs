defmodule Specter.MixProject do
  use Mix.Project

  @scm_url "https://github.com/livinginthepast/specter"
  @version "0.1.0"

  def project do
    [
      app: :specter,
      deps: deps(),
      description: description(),
      elixir: "~> 1.13",
      package: package(),
      homepage_url: @scm_url,
      package: package(),
      preferred_cli_env: [credo: :test, dialyzer: :test, docs: :dev],
      source_url: @scm_url,
      start_permanent: Mix.env() == :prod,
      version: @version
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

  defp description,
    do: """
    A rustler nif wrapping webrtc.rs.
    """

  defp package do
    [
      licenses: ["MIT"],
      maintainers: ["Eric Saxby"],
      links: %{"GitHub" => @scm_url},
      files: ~w(
        LICENSE.md
        README.md
        config
        lib
        mix.exs
        native
      )
    ]
  end
end
