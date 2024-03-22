#!/bin/bash

# List of badge IDs
declare -a badges=(
  # Discord Staff
  "5e74e9b61934fc1f67c65515d1f7e60d"
  # Partnered Server Owner
  "3f9748e53446a137a052f3454e2de41e"
  # HypeSquad Events
  "bf01d1073931f921909045f3a39fd264"
  # Discord Bug Hunter (Level 1)
  "2717692c7dca7289b35297368a940dd0"
  # HypeSquad Bravery
  "8a88d63823d8a71cd5e390baa45efa02"
  # HypeSquad Brilliance
  "011940fd013da3f7fb926e4a1cd2e618"
  # HypeSquad Balance
  "3aa41de486fa12454c3761e8e223442e"
  # Early Supporter
  "7060786766c9c840eb3019e725d2b358"
  # Discord Bug Hunter (Level 2)
  "848f79194d4be5ff5f81505cbd0ce1e6"
  # Early Verified Bot Developer
  "6df5892e0f35b051f8b61eace34f4967"
  # Moderator Programmes Alumni
  "fee1624003e2fee35cb398e125dc479b"
  # Active Developer
  "6bdc42827a38498929a4920da12695d9"
)

# Download and upload the badge
for i in "${badges[@]}"
do
  curl -O "https://cdn.discordapp.com/badge-icons/$i.png"
  aws s3 cp "$i.png" "s3://$1/badge-icons/$i.png"
done