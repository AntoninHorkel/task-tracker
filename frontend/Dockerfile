# FROM nixos/nix:latest
# WORKDIR /frontend
# COPY . .
# RUN nix build . --extra-experimental-features "nix-command flakes"
# EXPOSE 3000
# CMD ./result/bin/task-tracker-frontend
FROM node:25-alpine
WORKDIR /frontend
COPY . .
EXPOSE 3000
CMD npm install && npm run dev -- --host
