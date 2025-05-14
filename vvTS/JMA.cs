using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200017C RID: 380
	[HandlerCategory("vvAverages"), HandlerName("JMA")]
	public class JMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000C03 RID: 3075 RVA: 0x000343D8 File Offset: 0x000325D8
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("jma", new string[]
			{
				this.length.ToString(),
				this.phase.ToString(),
				src.GetHashCode().ToString()
			}, () => JMA.GenJMA(src, this.length, this.phase));
		}

		// Token: 0x06000C02 RID: 3074 RVA: 0x00033820 File Offset: 0x00031A20
		public static IList<double> GenJMA(IList<double> src, int _len, int _phase)
		{
			double[] array = new double[src.Count];
			double num = 0.0;
			double num2 = 0.0;
			double num3 = 0.0;
			double num4 = 0.0;
			int num5 = 0;
			int num6 = 0;
			int num7 = 0;
			int num8 = 0;
			double num9 = 0.0;
			double num10 = 0.0;
			double num11 = 0.0;
			double num12 = 0.0;
			double num13 = 0.0;
			double num14 = 0.0;
			double num15 = 0.0;
			double num16 = 0.0;
			double num17 = 0.0;
			int num18 = 0;
			double num19 = 0.0;
			double[] array2 = new double[500];
			double[] array3 = new double[500];
			double[] array4 = new double[500];
			double[] array5 = new double[500];
			for (int i = 0; i < _len; i++)
			{
				array[i] = src[i];
			}
			for (int j = _len; j < src.Count; j++)
			{
				double num20 = src[j];
				if (num18 < 61)
				{
					num18++;
					array5[num18] = num20;
				}
				if (num18 > 30)
				{
					double num21;
					if ((double)_len < 1.0000000002)
					{
						num21 = 1E-10;
					}
					else
					{
						num21 = (double)(_len - 1) / 2.0;
					}
					double num22;
					if (_phase < -100)
					{
						num22 = 0.5;
					}
					else if (_phase > 100)
					{
						num22 = 2.5;
					}
					else
					{
						num22 = (double)(_phase / 100) + 1.5;
					}
					double num23 = Math.Log(Math.Sqrt(num21));
					double num24 = num23;
					double num25;
					if (num23 / Math.Log(2.0) + 2.0 < 0.0)
					{
						num25 = 0.0;
					}
					else
					{
						num25 = num24 / Math.Log(2.0) + 2.0;
					}
					double num26 = num25;
					double y;
					if (0.5 <= num26 - 2.0)
					{
						y = num26 - 2.0;
					}
					else
					{
						y = 0.5;
					}
					double num27 = Math.Sqrt(num21) * num26;
					double num28 = num27 / (num27 + 1.0);
					num21 *= 0.9;
					double num29 = num21 / (num21 + 2.0);
					int k;
					double num31;
					if (num17 != 0.0)
					{
						num17 = 0.0;
						int num30 = 0;
						for (k = 1; k <= 29; k++)
						{
							if (array5[k + 1] != array5[k])
							{
								num30 = 1;
							}
						}
						num31 = (double)num30 * 30.0;
						if (num31 == 0.0)
						{
							num11 = num20;
						}
						else
						{
							num11 = array5[1];
						}
						num10 = num11;
						if (num31 > 29.0)
						{
							num31 = 29.0;
						}
					}
					else
					{
						num31 = 0.0;
					}
					k = 0;
					while ((double)k <= num31)
					{
						int num32 = 31 - k;
						double num33;
						if (k == 0)
						{
							num33 = num20;
						}
						else
						{
							num33 = array5[num32];
						}
						double num34 = num33 - num10;
						double num35 = num33 - num11;
						if (Math.Abs(num34) > Math.Abs(num35))
						{
							num24 = Math.Abs(num34);
						}
						else
						{
							num24 = Math.Abs(num35);
						}
						double num36 = num24;
						double num37 = num36 + 1E-10;
						if (num7 <= 1)
						{
							num7 = 127;
						}
						else
						{
							num7--;
						}
						if (num8 <= 1)
						{
							num8 = 10;
						}
						else
						{
							num8--;
						}
						if (num9 < 128.0)
						{
							num9 += 1.0;
						}
						num = num + num37 - array4[num8];
						array4[num8] = num37;
						double num38;
						if (num9 > 10.0)
						{
							num38 = num / 10.0;
						}
						else
						{
							num38 = num / num9;
						}
						double num40;
						int num41;
						if (num9 > 127.0)
						{
							double num39 = array3[num7];
							array3[num7] = num38;
							num40 = 64.0;
							num41 = Convert.ToInt32(num40);
							while (num40 > 1.0)
							{
								if (array2[num41] < num39)
								{
									num40 *= 0.5;
									num41 += Convert.ToInt32(num40);
								}
								else if (array2[num41] <= num39)
								{
									num40 = 1.0;
								}
								else
								{
									num40 *= 0.5;
									num41 -= Convert.ToInt32(num40);
								}
							}
						}
						else
						{
							array3[num7] = num38;
							if (num3 + num4 > 127.0)
							{
								num4 -= 1.0;
								num41 = Convert.ToInt32(num4);
							}
							else
							{
								num3 += 1.0;
								num41 = Convert.ToInt32(num3);
							}
							if (num3 > 96.0)
							{
								num5 = 96;
							}
							else
							{
								num5 = Convert.ToInt32(num3);
							}
							if (num4 < 32.0)
							{
								num6 = 32;
							}
							else
							{
								num6 = Convert.ToInt32(num4);
							}
						}
						num40 = 64.0;
						int num42 = Convert.ToInt32(num40);
						while (num40 > 1.0)
						{
							if (array2[num42] >= num38)
							{
								if (array2[num42 - 1] <= num38)
								{
									num40 = 1.0;
								}
								else
								{
									num40 *= 0.5;
									num42 -= Convert.ToInt32(num40);
								}
							}
							else
							{
								num40 *= 0.5;
								num42 += Convert.ToInt32(num40);
							}
							if (num42 == 127 && num38 > array2[127])
							{
								num42 = 128;
							}
						}
						if (num9 > 127.0)
						{
							if (num41 >= num42)
							{
								if (num5 + 1 > num42 && num6 - 1 < num42)
								{
									num2 += num38;
								}
								else if (num6 > num42 && num6 - 1 < num41)
								{
									num2 += array2[num6 - 1];
								}
							}
							else if (num6 >= num42)
							{
								if (num5 + 1 < num42 && num5 + 1 > num41)
								{
									num2 += array2[num5 + 1];
								}
							}
							else if (num5 + 2 > num42)
							{
								num2 += num38;
							}
							else if (num5 + 1 < num42 && num5 + 1 > num41)
							{
								num2 += array2[num5 + 1];
							}
							if (num41 > num42)
							{
								if (num6 - 1 < num41 && num5 + 1 > num41)
								{
									num2 -= array2[num41];
								}
								else if (num5 < num41 && num5 + 1 > num42)
								{
									num2 -= array2[num5];
								}
							}
							else if (num5 + 1 > num41 && num6 - 1 < num41)
							{
								num2 -= array2[num41];
							}
							else if (num6 > num41 && num6 < num42)
							{
								num2 -= array2[num6];
							}
						}
						if (num41 <= num42)
						{
							if (num41 >= num42)
							{
								array2[num42] = num38;
							}
							else
							{
								for (int l = num41 + 1; l <= num42 - 1; l++)
								{
									array2[l - 1] = array2[l];
								}
								array2[num42 - 1] = num38;
							}
						}
						else
						{
							for (int l = num41 - 1; l >= num42; l--)
							{
								array2[l + 1] = array2[l];
							}
							array2[num42] = num38;
						}
						if (num9 <= 127.0)
						{
							num2 = 0.0;
							for (int l = num6; l <= num5; l++)
							{
								num2 += array2[l];
							}
						}
						double num43 = num2 / (double)(num5 - num6 + 1);
						if (num19 + 1.0 > 31.0)
						{
							num19 = 31.0;
						}
						else
						{
							num19 += 1.0;
						}
						if (num19 <= 30.0)
						{
							if (num34 > 0.0)
							{
								num10 = num33;
							}
							else
							{
								num10 = num33 - num34 * num28;
							}
							if (num35 < 0.0)
							{
								num11 = num33;
							}
							else
							{
								num11 = num33 - num35 * num28;
							}
							num14 = num20;
							if (num19 == 30.0 && num19 == 30.0)
							{
								num15 = num20;
								double num44;
								if (Math.Ceiling(num27) >= 1.0)
								{
									num44 = Math.Ceiling(num27);
								}
								else
								{
									num44 = 1.0;
								}
								double num45 = Math.Ceiling(num44);
								if (Math.Floor(num27) >= 1.0)
								{
									num24 = Math.Floor(num27);
								}
								else
								{
									num24 = 1.0;
								}
								double num46 = Math.Ceiling(num24);
								double num47;
								if (num45 == num46)
								{
									num47 = 1.0;
								}
								else
								{
									num44 = num45 - num46;
									num47 = (num27 - num46) / num44;
								}
								int num30;
								if (num46 <= 29.0)
								{
									num30 = Convert.ToInt32(num46);
								}
								else
								{
									num30 = 29;
								}
								int num48;
								if (num45 <= 29.0)
								{
									num48 = Convert.ToInt32(num45);
								}
								else
								{
									num48 = 29;
								}
								num13 = (num20 - array5[num18 - num30]) * (1.0 - num47) / num46 + (num20 - array5[num18 - num48]) * num47 / num45;
							}
						}
						else
						{
							if (num26 >= Math.Pow(num36 / num43, y))
							{
								num23 = Math.Pow(num36 / num43, y);
							}
							else
							{
								num23 = num26;
							}
							if (num23 < 1.0)
							{
								num24 = 1.0;
							}
							else
							{
								if (num26 >= Math.Pow(num36 / num43, y))
								{
									num25 = Math.Pow(num36 / num43, y);
								}
								else
								{
									num25 = num26;
								}
								num24 = num25;
							}
							num12 = num24;
							double num49 = Math.Pow(num28, Math.Sqrt(num12));
							if (num34 > 0.0)
							{
								num10 = num33;
							}
							else
							{
								num10 = num33 - num34 * num49;
							}
							if (num35 < 0.0)
							{
								num11 = num33;
							}
							else
							{
								num11 = num33 - num35 * num49;
							}
						}
						k++;
					}
					if (num19 > 30.0)
					{
						double num50 = Math.Pow(num29, num12);
						num15 = (1.0 - num50) * num20 + num50 * num15;
						num16 = (num20 - num15) * (1.0 - num29) + num29 * num16;
						double num51 = num22 * num16 + num15;
						double num52 = -num50 * 2.0;
						double num53 = num50 * num50;
						double num54 = num52 + num53 + 1.0;
						num13 = (num51 - num14) * num54 + num53 * num13;
						num14 += num13;
					}
					array[j] = num14;
				}
				if (num18 <= 30)
				{
					array[j] = src[j];
				}
			}
			return array;
		}

		// Token: 0x170003F1 RID: 1009
		public IContext Context
		{
			// Token: 0x06000C04 RID: 3076 RVA: 0x00034456 File Offset: 0x00032656
			get;
			// Token: 0x06000C05 RID: 3077 RVA: 0x0003445E File Offset: 0x0003265E
			set;
		}

		// Token: 0x170003EF RID: 1007
		[HandlerParameter(true, "7", Min = "0", Max = "80", Step = "1")]
		public int length
		{
			// Token: 0x06000BFE RID: 3070 RVA: 0x000337FB File Offset: 0x000319FB
			get;
			// Token: 0x06000BFF RID: 3071 RVA: 0x00033803 File Offset: 0x00031A03
			set;
		}

		// Token: 0x170003F0 RID: 1008
		[HandlerParameter(true, "0", Min = "-100", Max = "100", Step = "25")]
		public int phase
		{
			// Token: 0x06000C00 RID: 3072 RVA: 0x0003380C File Offset: 0x00031A0C
			get;
			// Token: 0x06000C01 RID: 3073 RVA: 0x00033814 File Offset: 0x00031A14
			set;
		}
	}
}
