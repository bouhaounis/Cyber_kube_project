import { useEffect, useRef } from "react";
import * as THREE from "three";

export function Cluster3D() {
  const mountRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const mount = mountRef.current;
    if (!mount) return;

    const scene = new THREE.Scene();
    scene.background = new THREE.Color("#0b1220");

    const camera = new THREE.PerspectiveCamera(
      55,
      mount.clientWidth / mount.clientHeight,
      0.1,
      1000
    );
    camera.position.set(0, 2.2, 6.2);

    const renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setSize(mount.clientWidth, mount.clientHeight);
    renderer.setPixelRatio(window.devicePixelRatio);
    mount.appendChild(renderer.domElement);

    // Lights
    const hemi = new THREE.HemisphereLight(0xffffff, 0x223344, 0.8);
    scene.add(hemi);
    const dir = new THREE.DirectionalLight(0xffffff, 1.0);
    dir.position.set(4, 6, 2);
    scene.add(dir);

    // Ground
    const grid = new THREE.GridHelper(12, 12, 0x334155, 0x1f2937);
    (grid.material as THREE.Material).opacity = 0.35;
    (grid.material as THREE.Material).transparent = true;
    scene.add(grid);

    // Simple “nodes” (spheres) + “pods” (cubes)
    const nodeMat = new THREE.MeshStandardMaterial({ color: 0x38bdf8 });
    const podMat = new THREE.MeshStandardMaterial({ color: 0xa3e635 });
    const alertMat = new THREE.MeshStandardMaterial({ color: 0xf97316 });

    const group = new THREE.Group();
    scene.add(group);

    const nodes: THREE.Mesh[] = [];
    for (let i = 0; i < 5; i++) {
      const node = new THREE.Mesh(new THREE.SphereGeometry(0.35, 32, 32), nodeMat);
      node.position.set(-2.5 + i * 1.25, 0.35, 0);
      group.add(node);
      nodes.push(node);

      for (let p = 0; p < 3; p++) {
        const pod = new THREE.Mesh(new THREE.BoxGeometry(0.22, 0.22, 0.22), podMat);
        pod.position.set(node.position.x + (p - 1) * 0.35, 0.22, -1.0 + p * 0.45);
        group.add(pod);
      }
    }

    // Fake alert pulse on one node
    const alert = new THREE.Mesh(new THREE.TorusGeometry(0.55, 0.06, 16, 64), alertMat);
    alert.rotation.x = Math.PI / 2;
    alert.position.copy(nodes[2].position);
    group.add(alert);

    let raf = 0;
    const animate = () => {
      raf = requestAnimationFrame(animate);
      group.rotation.y += 0.0035;
      alert.scale.setScalar(1 + 0.15 * Math.sin(Date.now() / 200));
      renderer.render(scene, camera);
    };
    animate();

    const onResize = () => {
      if (!mount) return;
      camera.aspect = mount.clientWidth / mount.clientHeight;
      camera.updateProjectionMatrix();
      renderer.setSize(mount.clientWidth, mount.clientHeight);
    };
    window.addEventListener("resize", onResize);

    return () => {
      cancelAnimationFrame(raf);
      window.removeEventListener("resize", onResize);
      mount.removeChild(renderer.domElement);
      renderer.dispose();
    };
  }, []);

  return <div ref={mountRef} style={{ width: "100%", height: 520 }} />;
}

