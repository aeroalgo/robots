using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000050 RID: 80
	[HandlerCategory("vvIndicators"), HandlerDecimals(2), HandlerName("RMI")]
	public class RMI : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x060002DB RID: 731 RVA: 0x0000DB33 File Offset: 0x0000BD33
		public IList<double> Execute(IList<double> src)
		{
			return RMI.GenRMI(src, this.RMIperiod, this.MOMperiod);
		}

		// Token: 0x060002DA RID: 730 RVA: 0x0000D9E4 File Offset: 0x0000BBE4
		public static IList<double> GenRMI(IList<double> src, int rmiperiod, int momperiod)
		{
			int count = src.Count;
			int num = Math.Max(rmiperiod, momperiod);
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			for (int i = num; i < count; i++)
			{
				double num2 = 0.0;
				double num3 = 0.0;
				double num5;
				double num6;
				if (i == num)
				{
					for (int j = momperiod; j <= i; j++)
					{
						double num4 = src[j] - src[j - momperiod];
						if (num4 > 0.0)
						{
							num3 += num4;
						}
						else
						{
							num2 -= num4;
						}
					}
					num5 = num3 / (double)rmiperiod;
					num6 = num2 / (double)rmiperiod;
				}
				else
				{
					double num4 = src[i] - src[i - momperiod];
					if (num4 > 0.0)
					{
						num3 = num4;
					}
					else
					{
						num2 = -num4;
					}
					num5 = (array2[i - 1] * (double)(rmiperiod - 1) + num3) / (double)rmiperiod;
					num6 = (array3[i - 1] * (double)(rmiperiod - 1) + num2) / (double)rmiperiod;
				}
				array2[i] = num5;
				array3[i] = num6;
				if (num6 == 0.0)
				{
					array[i] = 0.0;
				}
				else
				{
					array[i] = 100.0 * num5 / (num5 + num6);
				}
			}
			return array;
		}

		// Token: 0x170000F8 RID: 248
		public IContext Context
		{
			// Token: 0x060002DC RID: 732 RVA: 0x0000DB47 File Offset: 0x0000BD47
			get;
			// Token: 0x060002DD RID: 733 RVA: 0x0000DB4F File Offset: 0x0000BD4F
			set;
		}

		// Token: 0x170000F7 RID: 247
		[HandlerParameter(true, "5", Min = "1", Max = "20", Step = "1")]
		public int MOMperiod
		{
			// Token: 0x060002D8 RID: 728 RVA: 0x0000D9D1 File Offset: 0x0000BBD1
			get;
			// Token: 0x060002D9 RID: 729 RVA: 0x0000D9D9 File Offset: 0x0000BBD9
			set;
		}

		// Token: 0x170000F6 RID: 246
		[HandlerParameter(true, "14", Min = "1", Max = "20", Step = "1")]
		public int RMIperiod
		{
			// Token: 0x060002D6 RID: 726 RVA: 0x0000D9C0 File Offset: 0x0000BBC0
			get;
			// Token: 0x060002D7 RID: 727 RVA: 0x0000D9C8 File Offset: 0x0000BBC8
			set;
		}
	}
}
