using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200015A RID: 346
	[HandlerCategory("vvAverages"), HandlerName("Bezier")]
	public class Bezier : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000AE3 RID: 2787 RVA: 0x0002CE14 File Offset: 0x0002B014
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("bezier", new string[]
			{
				this.Length.ToString(),
				this.Sense.ToString(),
				src.GetHashCode().ToString()
			}, () => Bezier.GenBezier(src, this.Length, this.Sense));
		}

		// Token: 0x06000AE2 RID: 2786 RVA: 0x0002CDAC File Offset: 0x0002AFAC
		private static double fact(int value)
		{
			double num = 1.0;
			for (double num2 = 2.0; num2 < (double)(value + 1); num2 += 1.0)
			{
				num *= num2;
			}
			return num;
		}

		// Token: 0x06000AE1 RID: 2785 RVA: 0x0002CD20 File Offset: 0x0002AF20
		public static IList<double> GenBezier(IList<double> src, int period, double sense)
		{
			int count = src.Count;
			double[] array = new double[count];
			for (int i = period; i < count; i++)
			{
				double num = 0.0;
				for (int j = period; j >= 0; j--)
				{
					num += src[i - j] * (Bezier.fact(period) / (Bezier.fact(j) * Bezier.fact(period - j))) * Math.Pow(sense, (double)j) * Math.Pow(1.0 - sense, (double)(period - j));
				}
				array[i] = num;
			}
			return array;
		}

		// Token: 0x1700039B RID: 923
		public IContext Context
		{
			// Token: 0x06000AE4 RID: 2788 RVA: 0x0002CE95 File Offset: 0x0002B095
			get;
			// Token: 0x06000AE5 RID: 2789 RVA: 0x0002CE9D File Offset: 0x0002B09D
			set;
		}

		// Token: 0x17000399 RID: 921
		[HandlerParameter(true, "9", Min = "3", Max = "50", Step = "1")]
		public int Length
		{
			// Token: 0x06000ADD RID: 2781 RVA: 0x0002CCFC File Offset: 0x0002AEFC
			get;
			// Token: 0x06000ADE RID: 2782 RVA: 0x0002CD04 File Offset: 0x0002AF04
			set;
		}

		// Token: 0x1700039A RID: 922
		[HandlerParameter(true, "0.5", Min = "0", Max = "1", Step = "0.01")]
		public double Sense
		{
			// Token: 0x06000ADF RID: 2783 RVA: 0x0002CD0D File Offset: 0x0002AF0D
			get;
			// Token: 0x06000AE0 RID: 2784 RVA: 0x0002CD15 File Offset: 0x0002AF15
			set;
		}
	}
}
