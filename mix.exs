defmodule Specter.MixProject do
  use Mix.Project

  @scm_url "https://github.com/synchronal/specter"
  @version "0.5.0"

  def project do
    [
      app: :specter,
      deps: deps(),
      description: description(),
      dialyzer: dialyzer(),
      docs: docs(),
      elixir: "~> 1.13",
      elixirc_paths: elixirc_paths(Mix.env()),
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
      {:jason, "~> 1.3"},
      {:markdown_formatter, "~> 1.0", only: :dev, runtime: false},
      {:mix_audit, "~> 2.0", only: [:dev], runtime: false},
      {:moar, "~> 1.7", only: [:test]},
      {:rustler, "~> 0.31"},
      {:uuid, "~> 1.1", only: [:test]}
    ]
  end

  defp description,
    do: """
    A rustler nif wrapping webrtc.rs.
    """

  defp dialyzer do
    [
      plt_add_apps: [:ex_unit, :mix],
      plt_add_deps: :app_tree,
      plt_core_path: "_build/#{Mix.env()}",
      plt_file: {:no_warn, "priv/plts/#{otp_version()}/dialyzer.plt"}
    ]
  end

  defp elixirc_paths(:test), do: ["lib", "test/support"]
  defp elixirc_paths(_), do: ["lib"]

  defp otp_version do
    Path.join([:code.root_dir(), "releases", :erlang.system_info(:otp_release), "OTP_VERSION"])
    |> File.read!()
    |> String.trim()
  end

  defp docs() do
    [
      main: "readme",
      extras: doc_extras(),
      groups_for_extras: groups_for_extras(),
      source_ref: "v#{@version}",
      assets: "guides/assets"
    ]
  end

  defp doc_extras() do
    [
      "README.md",
      "guides/lifecycle.md",
      "guides/internal_docs/README.md": [filename: "internal_readme", title: "README"],
      "guides/internal_docs/architecture.md": []
    ]
  end

  defp groups_for_extras() do
    [
      {"Guides", Path.wildcard("guides/*.md")},
      {"Internal docs", Path.wildcard("guides/internal_docs/*.md")}
    ]
  end

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
        native/specter_nif/src
        native/specter_nif/Cargo.toml
        native/specter_nif/Cargo.lock
      )
    ]
  end
end
