using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000155 RID: 341
	[HandlerCategory("vvAverages"), HandlerName("ALMA")]
	public class ALMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000AB7 RID: 2743 RVA: 0x0002C5D0 File Offset: 0x0002A7D0
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("alma", new string[]
			{
				this.ALMAperiod.ToString(),
				this.Sigma.ToString(),
				this.Offset.ToString(),
				src.GetHashCode().ToString()
			}, () => ALMA.GenALMA(src, this.ALMAperiod, this.Sigma, this.Offset));
		}

		// Token: 0x06000AB6 RID: 2742 RVA: 0x0002C4A4 File Offset: 0x0002A6A4
		public static IList<double> GenALMA(IList<double> src, int almaperiod, double sigma, double offset)
		{
			int count = src.Count;
			if (count < almaperiod)
			{
				return null;
			}
			double[] array = new double[count];
			double[] array2 = new double[count];
			double num = Math.Floor(offset * (double)(almaperiod - 1));
			double num2 = (double)almaperiod / sigma;
			double num3 = 0.0;
			for (int i = 0; i < almaperiod; i++)
			{
				array[i] = Math.Exp(-(((double)i - num) * ((double)i - num)) / (2.0 * num2 * num2));
				num3 += array[i];
			}
			for (int j = 0; j < almaperiod; j++)
			{
				array[j] /= num3;
			}
			for (int k = almaperiod; k < count; k++)
			{
				double num4 = 0.0;
				for (int l = 0; l < almaperiod; l++)
				{
					if (l < almaperiod)
					{
						num4 += array[l] * src[k - (almaperiod - 1 - l)];
					}
				}
				array2[k] = num4;
			}
			return array2;
		}

		// Token: 0x1700038A RID: 906
		[HandlerParameter(true, "9", Min = "3", Max = "50", Step = "1")]
		public int ALMAperiod
		{
			// Token: 0x06000AB0 RID: 2736 RVA: 0x0002C471 File Offset: 0x0002A671
			get;
			// Token: 0x06000AB1 RID: 2737 RVA: 0x0002C479 File Offset: 0x0002A679
			set;
		}

		// Token: 0x1700038D RID: 909
		public IContext Context
		{
			// Token: 0x06000AB8 RID: 2744 RVA: 0x0002C663 File Offset: 0x0002A863
			get;
			// Token: 0x06000AB9 RID: 2745 RVA: 0x0002C66B File Offset: 0x0002A86B
			set;
		}

		// Token: 0x1700038C RID: 908
		[HandlerParameter(true, "1", Min = "0", Max = "1", Step = "0.05")]
		public double Offset
		{
			// Token: 0x06000AB4 RID: 2740 RVA: 0x0002C493 File Offset: 0x0002A693
			get;
			// Token: 0x06000AB5 RID: 2741 RVA: 0x0002C49B File Offset: 0x0002A69B
			set;
		}

		// Token: 0x1700038B RID: 907
		[HandlerParameter(true, "6.0", Min = "1", Max = "10", Step = "1")]
		public double Sigma
		{
			// Token: 0x06000AB2 RID: 2738 RVA: 0x0002C482 File Offset: 0x0002A682
			get;
			// Token: 0x06000AB3 RID: 2739 RVA: 0x0002C48A File Offset: 0x0002A68A
			set;
		}
	}
}
