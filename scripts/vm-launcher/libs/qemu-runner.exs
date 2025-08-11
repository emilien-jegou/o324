defmodule QemuRunner do
  @moduledoc """
  A module for running an iso image
  """

  def launch(iso_path) do
    {output, exit_code} = System.cmd("qemu-system-x86_64", ["-enable-kvm", "-cpu", "host", "-m", "2048", "-netdev", "user,id=vmnet,hostfwd=tcp::2222-:22", "-device", "e1000,netdev=vmnet", "-display", "sdl", "-cdrom", iso_path, "-boot", "d",])

    case exit_code do
      0 -> {:ok}
      _ -> {:error, "Failed to run VM: #{output}"}
    end
  end
end


