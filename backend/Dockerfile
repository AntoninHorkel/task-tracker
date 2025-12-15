FROM nixos/nix:latest
WORKDIR /backend
COPY . .
RUN nix build . --extra-experimental-features "nix-command flakes"
EXPOSE 6767
CMD ./result/bin/task-tracker-backend
