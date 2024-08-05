# Ecole42Paris_KFS in rust

kvm 설치 \
cpu 가상화 기능 지원 확인
```sh
sudo apt install cpu-checker
kvm-ok
```
이렇게 나오면 사용가능
```sh
INFO: /dev/kvm exists
KVM acceleration can be use
```
kvm설치에 필요한 패키지
```sh
sudo apt install qemu-kvm libvirt-daemon-system libvirt-clients bridge-utils virtinst virt-manager

kvm --version
```
```text
qemu-kvm - KVM 하이퍼바이저에 대한 하드웨어 에뮬레이션을 제공
libvirt-daemon-system - libvirt 데몬을 시스템 서비스로 실행하는 구성 파일
libvirt-clients - 가상화 플랫폼을 관리하기위한 소프트웨어
bridge-utils - 이더넷 브리지를 구성하기위한 명령 줄 도구 세트
virtinst - 가상 머신을 만들기위한 명령 줄 도구 집합
virt-manager - 사용하기 쉬운 GUI 인터페이스와 libvirt를 통해 가상 머신을 관리하기위한 명령 줄 유틸리티
```