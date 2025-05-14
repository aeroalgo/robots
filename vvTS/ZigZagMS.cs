using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000073 RID: 115
	[HandlerCategory("vvIndicators")]
	public class ZigZagMS : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x06000416 RID: 1046 RVA: 0x00015CD0 File Offset: 0x00013ED0
		public IList<double> Execute(IList<double> src)
		{
			double[] array = new double[src.Count];
			double[] result = new double[src.Count];
			int num = 0;
			double num2 = this.ExtProcent * 0.01;
			for (int i = 0; i < src.Count; i++)
			{
				array[i] = 0.0;
			}
			double num3 = src[1];
			double num4 = num3;
			int num5 = 0;
			for (int j = 2; j < src.Count; j++)
			{
				if (src[j] > num4)
				{
					num4 = src[j];
					if (num5 != 2)
					{
						if (num4 - num3 >= num2 * num3)
						{
							num5 = 2;
							array[j] = num4;
							num = j;
							num3 = num4;
						}
						else
						{
							array[j] = 0.0;
						}
					}
					else
					{
						array[num] = 0.0;
						array[j] = num4;
						num = j;
						num3 = num4;
					}
				}
				else if (src[j] < num3)
				{
					num3 = src[j];
					if (num5 != 1)
					{
						if (num4 - num3 >= num2 * num4)
						{
							num5 = 1;
							array[j] = num3;
							num = j;
							num4 = num3;
						}
						else
						{
							array[j] = 0.0;
						}
					}
					else
					{
						array[num] = 0.0;
						array[j] = num3;
						num = j;
						num4 = num3;
					}
				}
				else
				{
					array[j] = 0.0;
				}
			}
			if (array[0] == 0.0)
			{
				array[0] = src[0];
			}
			if (this.ExtCurrentBar == 1 && array[src.Count - 1] == 0.0)
			{
				array[src.Count - 1] = src[src.Count - 1];
			}
			result = array;
			if (this.Direction == 1 && this.Interpolation == 0)
			{
				double[] array2 = new double[src.Count];
				double num6 = 0.0;
				for (int k = 0; k < src.Count; k++)
				{
					array2[k] = 0.0;
				}
				for (int l = 0; l < src.Count; l++)
				{
					if (array[l] != 0.0)
					{
						double num7 = array[l];
						for (int m = l - 1; m >= 0; m--)
						{
							if (array[m] != 0.0)
							{
								num6 = array[m];
								break;
							}
							num6 = 0.0;
						}
						if (num6 != 0.0)
						{
							if (num7 > num6)
							{
								array2[l] = 1.0;
							}
							else
							{
								array2[l] = -1.0;
							}
						}
					}
				}
				result = array2;
			}
			if (this.Direction == 0 && this.Interpolation == 1)
			{
				double[] array3 = new double[src.Count];
				double num8 = 0.0;
				double num9 = 0.0;
				double num10 = 0.0;
				int num11 = 0;
				for (int n = 0; n < src.Count; n++)
				{
					array3[n] = array[n];
				}
				for (int num12 = 0; num12 < src.Count; num12++)
				{
					if (array[num12] != 0.0)
					{
						double num13 = array[num12];
						int num14 = num12;
						for (int num15 = num12 - 1; num15 >= 0; num15--)
						{
							if (array[num15] != 0.0)
							{
								num8 = array[num15];
								num11 = num15;
								break;
							}
							num8 = array[0];
							num11 = 0;
						}
						if (num8 != 0.0)
						{
							num9 = (num13 - num8) / (double)(num14 - num11);
							num10 = num13 - num9 * (double)num14;
						}
						for (int num16 = num11 + 1; num16 < num14; num16++)
						{
							array3[num16] = num9 * (double)num16 + num10;
						}
					}
				}
				result = array3;
			}
			if (this.Direction == 1 && this.Interpolation == 1)
			{
				result = array;
			}
			return result;
		}

		// Token: 0x17000161 RID: 353
		[HandlerParameter(true, "0", Min = "0", Max = "1", Step = "1")]
		public int Direction
		{
			// Token: 0x06000412 RID: 1042 RVA: 0x00015CAE File Offset: 0x00013EAE
			get;
			// Token: 0x06000413 RID: 1043 RVA: 0x00015CB6 File Offset: 0x00013EB6
			set;
		}

		// Token: 0x17000160 RID: 352
		[HandlerParameter(true, "0", Min = "0", Max = "1", Step = "1")]
		public int ExtCurrentBar
		{
			// Token: 0x06000410 RID: 1040 RVA: 0x00015C9D File Offset: 0x00013E9D
			get;
			// Token: 0x06000411 RID: 1041 RVA: 0x00015CA5 File Offset: 0x00013EA5
			set;
		}

		// Token: 0x1700015F RID: 351
		[HandlerParameter(true, "0.3", Min = "0", Max = "10", Step = "0.1")]
		public double ExtProcent
		{
			// Token: 0x0600040E RID: 1038 RVA: 0x00015C8C File Offset: 0x00013E8C
			get;
			// Token: 0x0600040F RID: 1039 RVA: 0x00015C94 File Offset: 0x00013E94
			set;
		}

		// Token: 0x17000162 RID: 354
		[HandlerParameter(true, "0", Min = "0", Max = "1", Step = "1")]
		public int Interpolation
		{
			// Token: 0x06000414 RID: 1044 RVA: 0x00015CBF File Offset: 0x00013EBF
			get;
			// Token: 0x06000415 RID: 1045 RVA: 0x00015CC7 File Offset: 0x00013EC7
			set;
		}
	}
}
