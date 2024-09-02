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

grub 설치
```sh
sudo apt install grub-pc xorriso mtools
grub-mkrescue --version
```
grub이란 gnu프로젝트의 부트로더이고, 운영체제 대부분의 커널을 지원한다. 멀티부트로써 grub.cfg에 여러가지 운영체제 부팅파일을 설정할 수 있다. 거기에 플러스 cdrom에 올리는 iso파일 만드는 유틸까지 지원한다.


rust에서 지원하는 컴파일방식 target이 꽤있지만, 서브젝트에서 요구하는 32bit체제는 모두 지원종료하였기에 우리가 직접 target을 만들어서 설정해야한다.
```sh
rustc -Z unstable-options --print target-spec-json
```
```json
{
    "arch": "x86",  // 타겟 CPU 아키텍처 설정: x86 (32비트)
    "crt-objects-fallback": "false",  // CRT 오브젝트를 기본적으로 사용하지 않음
    "data-layout": "e-m:e-p:32:32-p270:32:32-p271:32:32-p272:64:64-i128:128-f64:32:64-f80:32-n8:16:32-S128",  // 메모리 및 레지스터 데이터 레이아웃 정의
    "disable-redzone": true,  // "red zone" 비활성화 (운영체제가 없는 환경에 적합)
    "features": "-mmx,-sse,+soft-float",  // CPU 기능 설정: MMX와 SSE 비활성화, 소프트웨어 부동소수점 사용
    "linker": "ld",  // 링커로 GNU ld 사용
    "linker-flavor": "gnu",  // 링커 플레버: GNU 스타일
    "llvm-target": "i386-unknown-none",  // LLVM 타겟 트리플
    "metadata": {
        "description": null,  // 설명 비어 있음
        "host_tools": null,  // 호스트 도구 비어 있음
        "std": null,  // 표준 라이브러리 비어 있음
        "tier": null  // 타겟의 지원 단계 비어 있음
    },
    "panic-strategy": "abort",  // 패닉 전략: abort (종료)
    "pre-link-args": {  // 링커에 전달할 사전 인수
        "gnu": [
            "-Tld-scripts/x86.ld",  // GNU 링커 스크립트 사용
            "-melf_i386"  // 32비트 ELF 형식 사용
        ],
        "gnu-lld": [
            "-Tld-scripts/x86.ld",  // LLD 링커 스크립트 사용
            "-melf_i386"  // 32비트 ELF 형식 사용
        ]
    },
    "target-pointer-width": "32"  // 포인터 크기: 32비트
}
```