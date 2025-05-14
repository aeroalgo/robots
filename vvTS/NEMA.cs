using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000164 RID: 356
	[HandlerCategory("vvAverages"), HandlerName("NEMA")]
	public class NEMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000B46 RID: 2886 RVA: 0x0002E399 File Offset: 0x0002C599
		public IList<double> Execute(IList<double> src)
		{
			return NEMA.GenNEMA(src, this.NemaPeriod, this.NemaDepth);
		}

		// Token: 0x06000B45 RID: 2885 RVA: 0x0002E370 File Offset: 0x0002C570
		private static double factorial(int n)
		{
			double num = 1.0;
			for (int i = 1; i <= n; i++)
			{
				num *= (double)i;
			}
			return num;
		}

		// Token: 0x06000B44 RID: 2884 RVA: 0x0002E22C File Offset: 0x0002C42C
		public static IList<double> GenNEMA(IList<double> src, int period, int depth)
		{
			int count = src.Count;
			double[] array = new double[count];
			double[,] array2 = new double[count, 50];
			double[] array3 = new double[50];
			depth = Math.Max(Math.Min(depth, 49), 1);
			for (int i = 1; i <= depth; i++)
			{
				array3[i] = NEMA.factorial(depth) / (NEMA.factorial(depth - i) * NEMA.factorial(i));
			}
			double num = 2.0 / (1.0 + (double)period);
			for (int j = 0; j < count; j++)
			{
				array[j] = 0.0;
				double num2 = src[j];
				double num3 = 1.0;
				int k = 0;
				while (k < depth)
				{
					if (j < 2)
					{
						array2[j, k] = num2;
					}
					else
					{
						array2[j, k] = array2[j - 1, k] + num * (num2 - array2[j - 1, k]);
					}
					num2 = array2[j, k];
					array[j] += num2 * num3 * array3[k + 1];
					k++;
					num3 *= -1.0;
				}
			}
			return array;
		}

		// Token: 0x170003B7 RID: 951
		public IContext Context
		{
			// Token: 0x06000B47 RID: 2887 RVA: 0x0002E3AD File Offset: 0x0002C5AD
			get;
			// Token: 0x06000B48 RID: 2888 RVA: 0x0002E3B5 File Offset: 0x0002C5B5
			set;
		}

		// Token: 0x170003B6 RID: 950
		[HandlerParameter(true, "10", Min = "1", Max = "50", Step = "1")]
		public int NemaDepth
		{
			// Token: 0x06000B42 RID: 2882 RVA: 0x0002E219 File Offset: 0x0002C419
			get;
			// Token: 0x06000B43 RID: 2883 RVA: 0x0002E221 File Offset: 0x0002C421
			set;
		}

		// Token: 0x170003B5 RID: 949
		[HandlerParameter(true, "10", Min = "1", Max = "50", Step = "1")]
		public int NemaPeriod
		{
			// Token: 0x06000B40 RID: 2880 RVA: 0x0002E208 File Offset: 0x0002C408
			get;
			// Token: 0x06000B41 RID: 2881 RVA: 0x0002E210 File Offset: 0x0002C410
			set;
		}
	}
}
